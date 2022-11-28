## Tangle Monitoring Stack

The monitoring stack consists of 
 - Prometheus
 - AlertManager
 - Loki
 - Promtail
 - Grafana
 - Node Exporter

 Running the monitoring stack requires that you are already running the tangle network node with atleast the following ports exports 

    - Prometheus : https://localhost:9615


The docker image starts all the above monitoring tools with the exception of Node exporter, node-exporter is ommitted since some metrics are not available when running inside a docker container.

Follow the instructions [here](https://prometheus.io/docs/guides/node-exporter/) to start the prometheus node exporter.

## Prerequisites

Before starting the monitoring stack, ensure the configs are setup correctly, 

 - (Optional) Set the `__SLACK_WEBHOOK_URL__` in `alertmanager.yml` to receive slack alerts
 - Ensure the promtail mount path matches your log directory

Note : All containers require connection to the localhost, this behaviour is different in Linux/Windows/Mac, the configs within the docker-compose and yml files assume a linux environment. Refer [this](https://stackoverflow.com/questions/24319662/from-inside-of-a-docker-container-how-do-i-connect-to-the-localhost-of-the-mach) to make necessary adjustments for your environment.

To start the monitoring stack, run

```bash
cd monitoring
docker compose up -d
```

You can then navigate to http://localhost:3000 to access the grafana dashboard

