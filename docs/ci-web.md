# CI web gate

The web job is intentionally stricter than a file-exists smoke check. It validates the real project subpath and the runtime module files that `index.html` imports before publication.
