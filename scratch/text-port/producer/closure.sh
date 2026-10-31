p(){ m=$1; if [[ $m == v2.* ]]; then echo "src/v2/${m#v2.}" | tr . / | sed 's/$/.dag/'; else echo "dag/$m" | tr . / | sed 's/$/.dag/'; fi; }
declare -A seen; q=("$@"); miss=()
while ((${#q[@]})); do m=${q[0]}; q=("${q[@]:1}"); [[ -n ${seen[$m]} ]] && continue; seen[$m]=1; f=$(p $m)
  if [[ ! -f $f ]]; then miss+=("$m"); continue; fi; echo "$f"
  for d in $(grep -oE '^import [A-Za-z0-9_.]+' "$f" | awk '{print $2}'); do [[ -z ${seen[$d]} ]] && q+=("$d"); done
done
for m in "${miss[@]}"; do echo "MISSING $m" >&2; done
