# Deployment regression

The first Pages workflow exported the project and checked only that `index.html` existed. The page then failed at runtime because it dynamically imported `pkg/sindri_causeway.js`, but the workflow had never built or copied the browser host. It also exported without `/fighting/` as the Pages base, so the import resolved against the organization-site root. The workflow now verifies both conditions before deployment.
