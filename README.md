# TabQ
Main website Monorepo

## Startpage
Quick access to links. Extendable with plugins.

## Magazines
Read magazines from Migros and Coop.

## Infomaniak Alias Manager
The Alias Manager is used to manage email aliases when there is a limited number of available alias addresses, such as with Infomaniak.
The tool manages a filter on a catch-all address that only allows registered aliases and moves everything else to the spam folder.
An alias can then be activated for sending as needed.

| Env | Description | Example |
| ---- | ---- | ---- |
| ALIAS_MANAGER_KEY | Cryptographic master key to encrypt Cookie. Must at least have 64 bytes | abc123 |

## API/Workflow
Update the static frontend without rebuilding the backend.

GET /refresh-from-compare: Compare latest tag and provided GITHUB_BRANCH and updated changed files<br>
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
| REPO_MAP | Map which folder from which repo should be considered | CMD-Golem/TabQ-Website;static/&VerticalLine;Other-User/Repo;src/ |
| LOCAL_MAP | Map where the files should be moved to relativ to PROD_DIR | CMD-Golem/TabQ-Website;static/&VerticalLine;Other-User/Repo;static/app1/ |