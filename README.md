<img src="brand/ada-judge.png" alt="ada-judge logo">

[![stars](https://img.shields.io/gitea/stars/oneprog/ada-judge?gitea_url=https://codeberg.org&style=for-the-badge&logo=codeberg)](https://codeberg.org/oneprog/ada-judge)
[![release](https://img.shields.io/gitea/v/release/oneprog/ada-judge?gitea_url=https://codeberg.org&style=for-the-badge)](https://codeberg.org/oneprog/ada-judge/releases)
[![last commit](https://img.shields.io/gitea/last-commit/oneprog/ada-judge?gitea_url=https://codeberg.org&style=for-the-badge)](https://codeberg.org/oneprog/ada-judge/commits/branch/master)

> Note: all AI-made PR's are banned from
> this project.

`ada-judge` is a fundamentally new competitive programming contests manager and solutions judger.

# Key features
- Built with 🦀 Rust: a blazingly fast and safe programming language
- Easy: problems are configured with TOML using [CLI](https://codeberg.org/oneprog/ada-judge-cli) and uploaded using [GUI](https://codeberg.org/oneprog/ada-judge-app)
- Powerful: supports different problems' types, including interactive and run-twice, subgroups' merging and per-test scoring
- Permissive license: licensed under the MIT license

# Getting started
At first, make sure that `docker` is installed in your system:
```bash
docker --version
```
Then, clone this repo:
```
git clone https://codeberg.org/oneprog/ada-judge
```
Go to the project's directory and create `.env` file with following fields:
```env
# Postgres profile data
POSTGRES_USER=
POSTGRES_PASSWORD=
POSTGRES_DB=
# Redis profile data
REDIS_PASSWORD=
REDIS_USER=
REDIS_USER_PASSWORD=
# Jwt settings
JWT_SECRET=
JWT_EXP_HOURS=
# Sandbox image name
SANDBOX_IMAGE=
# Number of parallel workers 
WORKERS_COUNT=
# Database url (for dev)
DATABASE_URL=postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:1111/${POSTGRES_DB}
```
After that, create `submissions_envs` directory.
Build and run `ada-judge` with `docker compose`:
```bash
docker compose build
DOCKER_GID=$(stat -c '%g' /var/run/docker.sock) docker compose up -d
```

Now, you can start using `ada-judge`!
