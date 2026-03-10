# ada-judge

> Note: all AI-made PR's are banned from
> this project.

ada-judge is a competitive programming contest manager and judger.

### Getting started
At first, make sure docker is installed in your system:
```bash
docker --version
```
Then, clone this repo:
```
git clone https://codeberg.org/oneprog/ada-judge
```
Cd to the project's directory, build and run ada-judge with docker compose:
```bash
docker compose build
docker compose up -d
```

> Note: remove the /target directory to
> make the build command run faster

Then, you can run test suite with `cargo test` to make sure everything was installed properly. 

Now, you can start using ada-judge!
