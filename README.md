<img src="brand/ada-judge.png" alt="ada-judge logo" width=720 height=400>

[![stars](https://img.shields.io/gitea/stars/oneprog/ada-judge?gitea_url=https://codeberg.org&style=for-the-badge&logo=codeberg)](https://codeberg.org/oneprog/ada-judge)
[![release](https://img.shields.io/gitea/v/release/oneprog/ada-judge?gitea_url=https://codeberg.org&style=for-the-badge)](https://codeberg.org/oneprog/ada-judge/releases)
[![last commit](https://img.shields.io/gitea/last-commit/oneprog/ada-judge?gitea_url=https://codeberg.org&style=for-the-badge)](https://codeberg.org/oneprog/ada-judge/commits/branch/master)

> Note: all AI-made PR's are banned from
> this project.

`ada-judge` is a competitive programming contests manager and solutions judger.

# Key features
- Built with 🦀 Rust: a blazingly fast and safe programming language
- Easy: problems configured with TOML and supports docker compose — no need for installing dependencies manually
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
# Password for registration
MASTER_PASSWORD=
# Jwt settings
JWT_SECRET=
JWT_EXP_HOURS=
# Sandbox image name for `submissions-judger`
SANDBOX_IMAGE=
# Number of parallel workers 
WORKERS_COUNT=
# Database url for tests
DATABASE_URL=postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:1111/${POSTGRES_DB}
```
After that, create `submissions_envs` directory.
Build and run `ada-judge` with `docker compose`:
```bash
docker compose build
docker compose up -d
```

Then, you can run test suite with `cargo test` to make sure that everything was installed properly. 

Now, you can start using `ada-judge`! 

`TODO`: «`ada-judge Book`»
