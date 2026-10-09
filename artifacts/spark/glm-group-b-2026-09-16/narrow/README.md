# Narrow inspect captures — `.HostConfig` and `.Config` per incarnation

These exist so a witness can read a container's creation state without parsing the whole
18 KB inspect document. `HostConfig` is 1.6–1.8 KB against 17.7 KB for the full document and
carries every field the transport question turns on.

**The two halves have DIFFERENT PROVENANCE and must not be read as one kind of evidence.**

## Fresh daemon output — the socket incarnation

`socket-rank0-*.json`, `socket-rank1-*.json` are verbatim stdout of

    docker inspect glm-native-tp4-socket --format '{{json .HostConfig}}'
    docker inspect glm-native-tp4-socket --format '{{json .Config}}'

taken 2026-09-17 on 192.168.1.236 and .237. This is the daemon answering a narrower
question — not a file cut down to fit a budget.

## Selected from the archive — the RoCE incarnation

`roce-rank0-*.json`, `roce-rank1-*.json` are the `HostConfig` and `Config` subtrees
**selected from** `../inspect-roce-rank0.json` / `-rank1.json` and re-serialised compactly.
They are NOT daemon output.

The reason is not convenience: **the RoCE containers no longer exist.** They were created
2026-09-16 ~20:00 UTC, and were destroyed and replaced later that evening by a recreate from
another session that did not carry the device mapping. No container on Group B carries
`Devices=1` today, so `docker inspect` cannot be asked this question again.

The selection is mechanically checkable against the archived full document, which is in this
same directory and unmodified. A consumer that needs daemon-emitted bytes for the RoCE side
does not have them and should say so rather than treating these as equivalent.

## What the pair still establishes

    socket   Devices=0  CapAdd=null            Ulimits=0  ShmSize=67108864
    roce     Devices=1  CapAdd=[CAP_IPC_LOCK]  Ulimits=1  ShmSize=67108864

Narrowing removed nothing the transport question turns on. `ShmSize` is identical across both,
which is the field that showed the launch spec had borrowed the pair route's 10 GiB figure.
