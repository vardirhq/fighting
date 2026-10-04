# Dynamic import failure

Observed symptom: the exported page reported that it could not fetch `sindri_causeway.js` from the organization-site root. The deployment had two defects: the export lacked the `/fighting/` base path and the Pages artifact did not contain the generated `pkg` browser host at all.
