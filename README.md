# CodeQuest
## Build-Instructions
### Prerequisites
The following programs need to be installed:
- Git
- GNU Make
- Docker (with compose and buildx support)
    - Docker BuildKit enabled (daemon settings file or DOCKER_BUILDKIT=1)

Build-Dependencies:
- OpenSSL
### Building from Source
1. Clone this repository `git clone https://github.com/Skulhunter5/codequest.git`
2. cd into it `cd codequest`
3. run `make docker_build`
4. run `make debug_quests`

Now all codequest-containers should be available in the local docker images. \
(You can check with `docker image ls`)
## Setup-Instructions
1. Create `.env` with the following template and fill in real passwords
    ```.env
    POSTGRES_USER=pguser
    POSTGRES_PASSWORD=pgpass
    POSTGRES_DB=codequest

    DB_USERNAME_USER_SERVICE=user_service
    DB_PASSWORD_USER_SERVICE=pgpass_user
    DB_USERNAME_QUEST_SERVICE=quest_service
    DB_PASSWORD_QUEST_SERVICE=pgpass_quest
    DB_USERNAME_PROGRESSION_SERVICE=progression_service
    DB_PASSWORD_PROGRESSION_SERVICE=pgpass_progression
    DB_USERNAME_STATISTICS_SERVICE=statistics_service
    DB_PASSWORD_STATISTICS_SERVICE=pgpass_statistics
    ```
2. Create secrets
    - Either run `make generate_secrets`
    - Or generate by hand
        - Create directory `./secrets/`
        - Create 32-byte base64-encoded, cryptographically secure, random material in `./secrets/secret_key` (e.g. using `head -c32 /dev/urandom | base64 > ./secrets/secret_key` or `openssl rand -base64 32 > ./secrets/secret_key`)
        - Create 4 to 64 bytes of base64-encoded (not padded) random material in `./secrets/salt` (e.g. using `head -c18 /dev/urandom | base64 > ./secrets/salt` or `openssl rand -base64 18 > ./secrets/salt`)
3. Start the docker compose stack: `docker compose up -d`
