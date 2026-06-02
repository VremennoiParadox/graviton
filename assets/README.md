# Demo assets

This folder holds terminal recordings for the README and portfolio.

## Files

| File | Purpose |
|------|---------|
| `demo.gif` | README hero animation (generate with VHS) |
| `demo.cast` | asciinema recording (optional) |
| `demo.tape` | VHS script (source of truth for GIF) |
| `screenshots/` | Static PNG captures |

## Generate the GIF (recommended: VHS)

Install [VHS](https://github.com/charmbracelet/vhs), then from the repo root:

```bash
./scripts/record-demo.sh
```

Or manually:

```bash
vhs assets/demo.tape
```

This writes `assets/demo.gif` using the scripted demo in `demo.tape`.

## Generate asciinema (optional)

```bash
asciinema rec assets/demo.cast
# run the demo commands, then exit
asciinema play assets/demo.cast
```

`demo.cast` is gitignored until you record it locally.

## Suggested demo flow

1. Earth–Moon TUI launch  
2. Zoom / pan  
3. Select Moon, toggle trails  
4. Toggle gravitational heatmap (`g`)  
5. Scenario switcher (`s`) → Solar System  
6. Time warp (`.`) and HUD energy drift  

See `demo.tape` for an automated subset.
