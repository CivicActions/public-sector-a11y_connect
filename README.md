# Public Sector A11yWatch: Rusty-A11y

A collection of dockerized server-side applications using the Rocketweb Rust Framework to connect CivicActions Gov accessibility scans to the A11yWatch API.

The application is built using Rust's package manager cargo and it can be built and run using Docker. The provided Dockerfile creates a container that includes all of the dependencies, source code, and the built binary of the application. When the container is run, it starts the application and it would be able to connect to the BigQuery using the environment variable GOOGLE_CLOUD_KEY.

## Quickstart

We :heart: :whale:

**Deploy with Docker**

Prerequisites:

- Set Github Repo Secret: `YOUR_GOOGLE_CLOUD_KEY` with perms

  - `bigquery.datasets.get`
  - `bigquery.tables.get`
  - `bigquery.tables.update`

**Run The Container**

```Dockerfile
docker run -e GOOGLE_CLOUD_KEY=<YOUR_GOOGLE_CLOUD_KEY> -p 8080:8080 <image-name>
```

**Docker Compose**
For those of you who prefer compose, here you go...

```Dockerfile
version: '3'
services:
  Rusty-A11y:
    build: .
    environment:
      - GOOGLE_CLOUD_KEY=${GOOGLE_CLOUD_KEY}
    ports:
      - "8080:8080"
```

## API Docs

Intro Text

### Authentication

`auth: x-token: ${{ secrets.token }}` (required) - The auth token to authenticate the request.

### Site

- **Check Status** `/up`
  This endpoint is used to check the reachability of websites. It sends a GET request to each domain and returns a JSON object containing a list of "DomainCheckResult" structs, which include the domain name and whether the site_error is true or false.

```curl
curl --request POST \
  --url http://localhost:8080/up \
  --header 'auth: x-token: ${{ secrets.token }}'
```

### System Status

- **Health Check** `/health`
  A way to check the health of Rusty-A11y and its dependencies, to ensure that everything is running as expected.

```curl
  curl --request GET \
    --url http://localhost:8080/health \
```

If Rusty-A11y returns a status code between 200 and 399, the probe is considered successful. If the container returns a status code between 400 and 599, the probe is considered failed.

- **Ready Check | Liveness Probe** `/ready`
  Used to determine if Rusty-A11y is ready to receive traffic and responsive. If the liveness probe fails, Kubernetes will restart the container. If the readiness probe fails, Kubernetes will stop sending traffic to Rusty-A11y

  ```curl
    curl --request GET \
      --url http://localhost:8080/ready \
  ```

  If Rusty-A11y returns a status code between 200 and 399, the probe is considered successful. If the container returns a status code between 400 and 599, the probe is considered failed.
