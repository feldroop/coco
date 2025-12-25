set shell := ["powershell.exe", "-c"]

run:
    tsc -p frontend/tsconfig.json --outDir frontend/out
    cargo run
