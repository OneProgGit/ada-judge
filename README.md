# ada-judge

> Note: all AI-made PR's are banned from
> this project.

ada-judge is a competitive programming contests manager and solutions judger.

### Getting started
At first, make sure docker is installed in your system:
```bash
docker --version
```
Then, clone this repo:
```
git clone https://codeberg.org/oneprog/ada-judge
```
Cd to the project's directory and create .env file with following fields:
```env
POSTGRES_USER=
POSTGRES_PASSWORD=
POSTGRES_DB=
REDIS_PASSWORD=
REDIS_USER=
REDIS_USER_PASSWORD=
DATABASE_URL=postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:1111/${POSTGRES_DB} # For tests
```
After that, create `submissions_envs` directory.
Build and run ada-judge with docker compose:
```bash
docker compose build
docker compose up -d
```

Then, you can run test suite with `cargo test` to make sure everything was installed properly. 

Now, you can start using ada-judge!
