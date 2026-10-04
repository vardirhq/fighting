# Build identity

The Pages export receives the short Git commit as its build ID. Sindri appends that identity to browser-host loading so a deployment can distinguish generated host revisions instead of quietly reusing an incompatible cached module.
