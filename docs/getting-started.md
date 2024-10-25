# Getting Started

First, [install Ethshadow and its dependencies](installation.md). Also, make sure `lighthouse`, `lcli`, `geth`, and
`bootnode` are available in your PATH environment variable. (TODO explain how to specify executable paths instead?)

Ethshadow uses, like Shadow, a yaml configuration file. Create a new File, e.g. `myfirstsim.yaml`.

In this file, you can specify any configuration option Shadow itself supports. There are [many options](https://shadow.github.io/docs/guide/shadow_config_spec.html),
we will focus on the essentials here. Add the following to your configuration:

```yaml
general:
  # How much time should we simulate?
  stop_time: 1h
  # Display a progress indicator?
  progress: true
```

These values will be passed to Shadow. Usually, when using Shadow directly, we would now specify our network topology
and hosts to simulate. However, Ethshadow does that for us. Ethshadow introduces several new configuration options,
contained in the `ethereum` section. In its most simple form, it looks like this:

```yaml
ethereum:
  # Distribute this many validators evenly across all nodes
  validators: 50
  # Create this many nodes with Geth, Lighthouse and a Validator client.
  # Additionally, a host with one boot node for CL and EL each is added.
  nodes: 10
```

That's it! After adding that, our simulation is ready to run. In a shall, move to the directory your config is in and
invoke:

```sh
ethshadow myfirstsim.yaml
```

The first run might take a moment, as Docker will have to pull an image. After some time, `Starting Shadow 3.2.0` will 
be logged, and the simulation will begin. Notice how the Simulation will run at variable speed: it will likely hang
for a moment at `00:00:04.999`, because all nodes start after giving the boot node five seconds to prepare. As genesis o
ccurs at `00:05:00.000`, time will pass relatively quickly until then, as nodes only search for peers and wait for
genesis. At approximately `00:05:12.000`, simulation will take a bit, as the first block is built and all nodes verify
it.

While waiting for the simulation to finish, note that a `data` directory was created next to your configuration file.
Feel free to look around in it. For each node, the clients' data directories are included. You can observe the
simulation by opening client logs contained within and following as the log gets written. As these logs tend to be
a bit noisy, you might also want to check the `shadow` subdirectory, which contains files where the stdout and stderr
of each process is redirected to. Here, you can easily check whether the simulation works by checking for error
messages and skipped slots.

Feel free to let the simulation finish or cancel it with Ctrl-C.

Let's take a look at a more sophisticated example (`sophisticated.yaml`):

```yaml
general:
  stop_time: 1h
  progress: true

ethereum:
  validators: 60
  nodes:
    - location: europe
      reliability: reliable
      tag: boot
      clients:
        el: geth_bootnode
        cl: lighthouse_bootnode
    - locations:
        - europe
        - na_east
        - na_west
      reliabilites:
        - reliable
        - home
      count:
        per_combination: 5
```

As you can see, we replaced the simple node count with a list of node specifications. Here, the yaml list has two
items. 

In the first one, we define a host located in europe, with a reliable internet connection. We also specify that
a Geth bootnode and a Lighthouse bootnode shall be run on that node.

In the second  one, we actually specify multimple nodes: notice the `count` property, which specifies five nodes per
combination. Combination here means every possible pair of specified locations and reliabilities: `europe` with
`reliable`, `europe` with `home`, `na_east` with `reliable`, and so on. As there are `2 * 3 = 6` combinations,
a total of `5 * 6 = 30` nodes will be created. As we specified 60 validators, each node will host 2 validators.

But what is a "location" and a "reliability"? In Ethereum, we have a lot of globally distributed nodes. Therefore,
we want to be able to simulate with varying latency between nodes. There are 8 built-in regions: `australia`,
`east_asia`, `europe`, `na_east`, `na_west`, `south_america`, `south_aftica`, and `west_asia`. Ethshadow has a table
with estimated latencies between these regions and will generate a network topology to make Shadow apply these
latencies to the traffic between the nodes.

Reliabilities seek to simulate the varying connection qualities available to nodes. As home stakers are important to
Ethereum, we want to include them into our simulations. The following reliabilities are available:

| Name        | Bandwidth (up and down) | Added latency | Added packet loss |
|-------------|-------------------------|---------------|-------------------|
| reliable    | 1 Gbit/s                | 0ms           | 0%                |
| home        | 50 Mbit/s               | 20ms          | 0.1%              |
| laggy       | 50 Mbit/s               | 300ms         | 5%                |
| constrained | 5 Mbit/s                | 20ms          | 0.1%              |
| bad         | 2 Mbit/s                | 500ms         | 20%               |

You can define your own locations and reliabilities as well as override the default values of the existing ones.

Before we can start a simulation with our more sophisticated simulation, we have to either delete the `data` directory
from the previous run or specify another directory:

```sh
ethshadow -d data_sophisticated sophisticated.yaml
```

Congrats! These are the basics of Ethshadow.
