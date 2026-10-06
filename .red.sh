set -uo pipefail
# usage: red.sh <mode> <witness file>
mode=$1; f=$2
case $mode in
  board_refusal_removed) python3 - <<'P'
p='dag/gunbc/host/baseboard_dimm_figure.dag'; s=open(p).read()
a='    GigabyteMp72Hb0 => BoardFigureUnmodeled { board: board }\n    AsrockRackAltrad8ud1l2t => BoardFigureUnmodeled { board: board }'
assert s.count(a)==1
s=s.replace(a,'    GigabyteMp72Hb0 => BaseboardDimmFigureOf { board: board, figure: MtCollinsGettingStartedGuideFigures { guide: mt_collins_gsg_authority } }\n    AsrockRackAltrad8ud1l2t => BaseboardDimmFigureOf { board: board, figure: MtCollinsGettingStartedGuideFigures { guide: mt_collins_gsg_authority } }')
open(p,'w').write(s)
P
  ;;
  board_not_read_from_binding) python3 - <<'P'
p='dag/gunbc/machine_intake/dimm_physical_orientation.dag'; s=open(p).read()
a='match baseboard_dimm_figure(board: binding.observation.identity.baseboard) {'
assert s.count(a)==1
s=s.replace(a,'match baseboard_dimm_figure(board: AmpereMtCollins) {').replace('import extdeps.boards.types { BaseboardModel }','import extdeps.boards.types { BaseboardModel, AmpereMtCollins }')
open(p,'w').write(s)
P
  ;;
  seal_removed) sed -i 's/^type DimmOrientedHost sole_constructor {/type DimmOrientedHost {/' dag/gunbc/machine_intake/dimm_physical_orientation.dag; grep -n '^type DimmOrientedHost' dag/gunbc/machine_intake/dimm_physical_orientation.dag ;;
  fleet_yml) python3 - <<'P'
s=open('.github/workflows/fleet-converge.yml',encoding='utf-8').read()
esc=s.replace('\\','\\\\').replace('"','\\"').replace('\n','\\n')
open('dag/test/claim/zz_scratch_fleet_yml_probe_test.dag','w',encoding='utf-8').write(
'module test.claim.zz_scratch_fleet_yml_probe\n\nimport std.types { Bool, String }\nimport gunbc.fleet_converge_workflow { expected_fleet_converge_yml, FleetConvergeYamlGenerated, FleetConvergeYamlGenerationRefused }\n\n'
'data committed_fleet_converge_yml: String = "'+esc+'"\n\n'
'fn generated() -> String {\n  match expected_fleet_converge_yml() {\n    FleetConvergeYamlGenerated { content } => content\n    FleetConvergeYamlGenerationRefused { reason: _ } => ""\n  }\n}\n\n'
'test fn fleet_converge_yml_regenerates_byte_identical() -> Bool {\n  let g = generated()\n  g != "" && g == committed_fleet_converge_yml\n}\n\n'
'test fn control_one_byte_off_is_not_equal() -> Bool {\n  generated() == concat(committed_fleet_converge_yml, "x")\n}\n')
print(len(s))
P
  ;;
esac
git --no-pager diff --stat -- dag
bash ./.run2.sh $f
