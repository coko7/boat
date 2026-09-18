Output demo.gif
Output demo.mp4

Require echo
Require boat
Require jq

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 600

Sleep 1s

Type "boat --help" Sleep 500ms  Enter
Sleep 3s Ctrl+l Sleep 500ms

Type "boat new 'make boat VHS' -t cli,charm --start" Sleep 500ms Enter
Sleep 4s

Type "boat get" Sleep 500ms Enter
Sleep 3s

Type "boat get --json | jq" Sleep 500ms Enter
Sleep 3s
Ctrl+l Sleep 500ms

Type "boat list" Sleep 500ms Enter
Sleep 4s
Ctrl+l Sleep 500ms

Type "boat ls --period today" Sleep 500ms Enter
Sleep 4s
Ctrl+l Sleep 500ms

Type "boat l --summary -p last-month" Sleep 500ms Enter
Sleep 4s
Ctrl+l Sleep 500ms

Type "boat l -s -p lmo -j | jq" Sleep 500ms Enter
Sleep 4s
Ctrl+l Sleep 500ms

Type `VHS_TASK=$(boat ls -saj | jq 'map(select(.name | contains("VHS"))) | .[0]')` Sleep 500ms Enter
Sleep 1s

Type "jq <<< $VHS_TASK" Sleep 500ms Enter
Sleep 4s

Type "VHS_TASK_ID=$(jq .id <<< $VHS_TASK)" Sleep 500ms Enter
Sleep 1s

Type "boat modify $VHS_TASK_ID --description 'work on demo.tape VHS gif for boat'" Sleep 500ms Enter
Sleep 3s

Type "boat get" Sleep 500ms Enter
Sleep 4s

Type "boat pause" Sleep 500ms Enter
Sleep 2s

Type "boat delete $VHS_TASK_ID" Sleep 500ms Enter
Sleep 5s
