# Capture Metrics

Currently, metrics are captured for Teku and Lighthouse. To capture metrics, simply add a node with the `prometheus` 
client to your configuration. **IMPORTANT:** currently, only nodes mentioned before the Prometheus node are considered
for monitoring.

```yaml
ethereum:
  nodes:
    ...monitored nodes here...
    - location: europe
      reliability: reliable
      tag: monitoring
      clients:
        monitoring: prometheus
```

To read the metrics after the simulation, simply start Prometheus, for example like this:

```shell
prometheus --storage.tsdb.path=<data_dir>/<node_subdir>/prometheus --storage.tsdb.retention.time=30y --config.file=/dev/null
```

You can use the Prometheus server as usual, for example by connecting a Grafana instance. Note that Shadow is always 
starting simulations at simulated time 01-01-2000 00:00 UTC.