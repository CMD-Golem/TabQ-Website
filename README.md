# TabQ
Main website Monorepo

## Startpage
Quick access to links. Extendable with plugins.

## Alias
Quickly exchange Infomaniak email addresses.<br>
Work in progress.

## Magazines
Read magazines from Migros and Coop.

## API/Workflow
Update the static frontend without rebuilding the backend.
GET /refresh-from-compare: Compare latest tag and provided GITHUB_BRANCH and updated changed files
POST /refresh-from-webhook: Listen with GITHUB_WEBHOOK for pushes to GITHUB_BRANCH and update changed files

| Env | Description | Example |
| ---- | ---- | ---- |
| AUTO_FETCH | Automatically run compare api after restart | true |
| COMPARE_API_BEARER | Bearer to authenticate compare api | abc123 |
| GITHUB_WEBHOOK_SECRET | Secret defined in the Github Webhook for detecting pushes | abc123 |
| GITHUB_USER_AGENT | User Agent used in Github API calls | Awesome-Octocat-App |
| GITHUB_BRANCH | Branch from which the data is loaded | main |
| TEMP_DIR | Local server dir to store downloaded files temporarly | tmp-static/ |
| PROD_DIR | Local server dir where updateable files are stored | static/ |
| REPO_MAP | Map which folder from which repo should be considered | CMD-Golem/TabQ-Website@static/&VerticalLine;Other-User/Repo@src/ |
| LOCAL_MAP | Map where the files should be moved to relativ to PROD_DIR | CMD-Golem/TabQ-Website@static/&VerticalLine;Other-User/Repo@static/app1/ |