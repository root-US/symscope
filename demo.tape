# demo.tape — generates docs/demo.gif.
#
# From the repo root, with the release binary already built:
#   cargo build --release
#   docker run --rm -v "$PWD":/vhs ghcr.io/charmbracelet/vhs demo.tape
#
# (The Docker image bundles ttyd/ffmpeg/chromium but has no cargo, so the
# binary is built on the host first and the hidden step just puts it on PATH.)
Output docs/demo.gif

Set FontSize 18
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 55ms

# hidden setup (not shown in recording)
Hide
Type "export PATH=$PWD/target/release:$PATH && clear"
Enter
Show

Type "symscope def Index"
Sleep 400ms
Enter
Sleep 2s

Type "symscope callers from_edges"
Sleep 400ms
Enter
Sleep 2s

Type "symscope graph build --depth 3"
Sleep 400ms
Enter
Sleep 3s

Type "symscope graph build --format mermaid"
Sleep 400ms
Enter
Sleep 3s
