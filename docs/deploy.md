# Deploy gating

Only non-pull-request runs configure and upload GitHub Pages and enter the deploy job. Pull requests still execute the build and verification stages, so deployment-specific mistakes are review-time failures rather than post-merge surprises.
