# Installation

Only Linux is supported. For more details, see the 
[Shadow documentation](https://shadow.github.io/docs/guide/supported_platforms.html).

1. [Install Shadow and its dependencies](https://shadow.github.io/docs/guide/install_dependencies.html). Shadow should be available in your PATH environment variable.
2. [Install Docker](https://docs.docker.com/engine/install/) and [make sure your user is allowed to manage Docker](https://docs.docker.com/engine/install/linux-postinstall/#manage-docker-as-a-non-root-user). 
The Docker daemon must be running while `ethshadow` prepares the simulation.
3. Install `ethshadow` by running `cargo install --path .`
4. Ensure that all clients you want to use in the simulation are installed, see each client's page for notes.