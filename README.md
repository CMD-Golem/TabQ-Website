# TabQ
Main website Monorepo

## Home
Quick access to links. Extendable with plugins.

## Alias
Quickly exchange the Infomaniak email address.<br>
Work in progress.

## Magazines
Read magazines from Migros and Coop.

## API/Workflow
Update the static frontend without rebuilding the backend.

| Env | Description | Example |
| ---- | ---- | ---- |
| GITHUB_WEBHOOK_SECRET | Secret defined in the Github Webhook | abc123 |
| GITHUB_BRANCH | Branch from which the data is loaded | main |
| TEMP_DIR | Local server dir to store downloaded files temporarly | /tmp-static/ |
| PROD_DIR | Local server dir where updateable files are stored | /static/ |
| REPO_MAP | Map which folder from which repo should be updated | CMD-Golem/TabQ-Website;static/&VerticalLine;Other-User/Repo;src/ |