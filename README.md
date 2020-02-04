# RAMP 

Distributed multiagent system written in Rust

To run our system just go to `ramp` directory and type:

```bash
# runs server for global synchronization
cargo run --bin server Server.toml

# runs hosts
cargo run --example fun-opt CoordSettings.toml SimulationSettings.toml
cargo run --example fun-opt Settings1.toml SimulationSettings.toml
cargo run --example fun-opt Settings2.toml SimulationSettings.toml 
```
localhost:9898 - metrics exposed by host - visualized by 3rd party systems (see below)

If you want to monitor system work (not only see results at the end) go to `promviz` directory and type:

```bash
# runs 3rd party monitoring systems 
docker-compose -f full-compose.yaml up
```
localhost:9090 - prometheus

localhost:8080 - vizceral

localhost:3000 - grafana

You can change `fun-opt` to `ecosys` problem. Just change `fun-opt` in the above commands to `ecosys` and `SimulationSettings.toml` to `WS_SimulationSettings.toml`. You can also disable global synchronization mechanism by changing value of variable `sync` to `false` in each settings file.

After starting grafana you have to import dashboards from `promviz/dashboards` directory. 
To see results in `node-exporter` dashboard you have to install and run [node_exporter](https://github.com/prometheus/node_exporter) on your own.

[how to import dashborad](https://grafana.com/docs/grafana/latest/reference/export_import/#importing-a-dashboard)

