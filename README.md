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
Cd to the project's directory and build the container:
```bash
docker build -t sandbox-runner .
```
Then, you can run test suite with `cargo test` to make sure everything was installed properly. 

Now, you can start using ada-judge!
