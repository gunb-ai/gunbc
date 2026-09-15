# File-backed Engram storage for vLLM d2d649e674c75425d2d6975c87eb89fd4d55fff8.
#
# THIS IS A BACKING SWAP, NOT NEW PREFETCH MACHINERY. Upstream
# vllm/models/deepseek_v4_1/nvidia/engram.py already has Engram._init_staging,
# prepare_embeddings, _start_prefetch / _finish_prefetch / _ready_rows on a
# dedicated cuda Stream, and ParallelEngramEmbedding._allocate_weights.
# DPSharedEngramStorage mmaps /dev/shm and cudaHostRegister()s the entire
# weight+scale allocation, then asserts tensor.is_pinned(). /dev/shm is tmpfs
# and on a unified-memory host that is the same pool as the accelerator.
#
# This module keeps the staging buffer, the stream, and the prefetch protocol.
# It changes where the rows come from: a rank-local file whose identity is
# verified at open, mmap'd MAP_PRIVATE without registering the whole table
# with CUDA. Token-time gathers copy requested rows into the existing bounded
# pinned staging buffer. A missing local path refuses; there is no NFS arm.
#
# MAGIC is the realization of gunbc.spark.v41_engram_row_store v41_row_store_magic.
# Do not mint a second revision or endianness constant here: those live on the
# encoding spec. Open verifies whole-file SHA against rank_digest_hex and MAGIC.
# encoding_digest / rank / format_revision are not on this type until a modeled
# header carries them — claiming them on ExpectedRowStoreIdentity without a
# check was a lying identity surface.

from __future__ import annotations

import hashlib
import mmap
import os
from dataclasses import dataclass
from typing import Tuple

import torch


# gunbc.spark.v41_engram_row_store v41_row_store_magic
MAGIC = b"ENGR1\n"


class EngramRowStoreIdentityMismatch(RuntimeError):
    pass


class EngramLocalStoreMissing(RuntimeError):
    pass


class EngramFullTablePinned(RuntimeError):
    pass


@dataclass(frozen=True)
class ExpectedRowStoreIdentity:
    rank_digest_hex: str


def _sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


class FileBackedEngramStorage:
    """Rank-local row store. The file is identity, not a path alias."""

    def __init__(self, path: str, expected: ExpectedRowStoreIdentity):
        if not os.path.isfile(path):
            raise EngramLocalStoreMissing(
                f"rank-local Engram row store missing at {path}; "
                "token-time NFS fallback is refused"
            )
        opened = _sha256_file(path)
        if opened != expected.rank_digest_hex:
            raise EngramRowStoreIdentityMismatch(
                f"opened {path} digest {opened}, expected rank digest "
                f"{expected.rank_digest_hex}"
            )
        self.path = os.path.abspath(path)
        self.expected = expected
        self._fd = os.open(self.path, os.O_RDONLY)
        self._mmap = mmap.mmap(self._fd, 0, access=mmap.ACCESS_READ)
        if self._mmap[: len(MAGIC)] != MAGIC:
            raise EngramRowStoreIdentityMismatch("row-store MAGIC mismatch")
        self._header_ok = True

    def close(self) -> None:
        self._mmap.close()
        os.close(self._fd)

    def fetch_rows(self, offsets: list[int], nbytes: int, dest: torch.Tensor) -> None:
        """Copy selected rows into an already-allocated bounded staging tensor."""
        if dest.is_pinned() and dest.numel() * dest.element_size() >= self._mmap.size():
            raise EngramFullTablePinned(
                "staging destination covers the whole table; this runtime refuses "
                "full-Engram pinned allocation"
            )
        view = memoryview(self._mmap)
        for i, off in enumerate(offsets):
            dest[i].view(torch.uint8).view(-1)[:nbytes].copy_(
                torch.frombuffer(view[off : off + nbytes], dtype=torch.uint8)
            )


def allocate_file_backed(
    path: str,
    expected: ExpectedRowStoreIdentity,
    staging_rows: int,
    row_bytes: int,
    device: torch.device,
) -> Tuple[FileBackedEngramStorage, torch.Tensor]:
    storage = FileBackedEngramStorage(path, expected)
    staging = torch.empty((staging_rows, row_bytes), dtype=torch.uint8, pin_memory=True)
    if staging.numel() >= os.path.getsize(path):
        raise EngramFullTablePinned(
            "pinned staging is not smaller than the row store; refusing full-table pin"
        )
    return storage, staging
