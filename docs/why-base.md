# Why `--base` matters

The exported page imports the browser host relatively. GitHub Pages serves this repository under `/fighting/`, so the page needs `<base href="/fighting/">`. Without that, a browser can request the correct-looking filename from the wrong directory and report only a dynamic-import failure.
