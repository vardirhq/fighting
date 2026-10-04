# Release flow

Pull requests run the Pages build as a non-deploying validation. Merges to `main` run the same build and then upload/deploy the generated artifact. This keeps the thing reviewed in CI materially closer to the thing served to players.
