# Runtime file verification

The Pages job checks the generated JavaScript and Wasm files by exact filename because `index.html` references that host package. If Sindri changes the package name in a future pinned revision, this check should fail and force the external project workflow to update deliberately.
