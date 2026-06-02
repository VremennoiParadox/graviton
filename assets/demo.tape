# orrery-tui README demo — render with: vhs assets/demo.tape
# Requires: vhs (https://github.com/charmbracelet/vhs), built orrery-tui binary

Output assets/demo.gif
Set Shell "bash"
Set FontSize 16
Set Width 1280
Set Height 720
Set Theme "Catppuccin Mocha"
Set Padding 24

Hide
Type "cargo build --release 2>/dev/null"
Enter
Sleep 2s
Show

Type "./target/release/orrery-tui run scenarios/earth-moon.toml"
Enter
Sleep 4s

# Pause and toggle trails
Type " "
Sleep 500ms
Type "T"
Sleep 1s

# Heatmap
Type "g"
Sleep 2s

# Zoom frame
Type "F"
Sleep 2s

# Help overlay
Type "?"
Sleep 2s
Type "?"
Sleep 1s

Type "q"
Sleep 500ms

Hide
Type "echo 'Demo complete — see docs/ for HORIZONS & Barnes–Hut'"
Enter
Sleep 1s
