# Pages concurrency

PR validation runs use their own concurrency group while actual deploys serialize together. A pull-request build therefore cannot evict a queued `main` deployment, matching the external-project lesson already learned in Mujaffa.
