# Public Sector A11yWatch: Rusty-A11y

A collection of dockerized server-side applications using the Rocketweb Rust Framework to connect CivicActions Gov accessibility scans to the A11yWatch API.

The application is built using Rust's package manager cargo and it can be built and run using Docker. The provided Dockerfile creates a container that includes all of the dependencies, source code, and the built binary of the application. When the container is run, it starts the application and it would be able to connect to the BigQuery using the environment variable GOOGLE_CLOUD_KEY.

## Getting Started

## Docs

The documentation can be found in the [docs](docs) folder.

## Contributing

## GOALS OF PROGRAM

We are deploying (another un-related project) the A11yWatch API to perform accessibility scans on websites.

We want to streamline the process of sending requests to that API by utilizing an intermediary solution which

1. Sends requests to the A11y API when requests are sent to this program
2. Waits for the A11y API/other system to reply
3. Apply out custom schema(s) to the reply data
4. Record the result in Google BigQuery
5. Respond to the requestor as needed

Think of this as a pass-through API. We make calls to it and it forwards those to external services. Those external services respons, it re-maps the data, and handles accordingly.

The API/docs I've started are below, but here is an explination of what we are looking for specifically.

**Variables for Docker Container**

|               Variable |                         Description                         |                                         Example                                         |
| ---------------------: | :---------------------------------------------------------: | :-------------------------------------------------------------------------------------: | --- |
|               A11Y_URL |                    URL of A11yWatch API                     |                            `https://api.a11ywatch.com/api/`                             |
| GOOGLE_APPLICATION_CREDENTIALS |                       JSON Google Key                       | [Google Docs](https://cloud.google.com/iam/docs/creating-managing-service-account-keys) |
| GOOGLE_SERVICE_ACCOUNT |                        Account Name                         |                    `someThing@someNamespace.iam.gserviceaccount.com`                    |
|      GOOGLE_PROJECT_ID |               project id for google big query               |
|         port_container |              Port exposed by this application               |                                          8080                                           |
|              port_host |           Port Docker maps to the Container Port            |                                           80                                            |
|                API_KEY | API key needed to access application. See Main.rs for notes |                                 `CGPk5x72BIwcaWVV7RWs`                                  |
|               A11Y_JWT |               JWT needed to access `A11Y_URL`               |                                                                                         | `   |
|         status_webhook |              A webhook to send status updates               |                                   https://webhook.com                                   |

## Mapping Files

### Website Health Check

User sends POST request to

### Accessibility Scan

### Accessibility Crawl

### Website Tech Inspect

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

- **Crawl Website** `/crawl`

- **Scan Website** `/crawl`

- **Inspect Website** `/inspect`

### System Status

- **Health Check** `/health`
  A way to check the health of the project and its dependencies, to ensure that everything is running as expected.

```curl
  curl --request GET \
    --url http://localhost:8080/health \
```

If it returns a status code between 200 and 399, the probe is considered successful. If the container returns a status code between 400 and 599, the probe is considered failed.

- **Ready Check | Liveness Probe** `/ready`
  Used to determine if it is ready to receive traffic and responsive. If the liveness probe fails, Kubernetes will restart the container. If the readiness probe fails, Kubernetes will stop sending traffic

  ```curl
    curl --request GET \
      --url http://localhost:8080/ready \
  ```

  If it returns a status code between 200 and 399, the probe is considered successful. If the container returns a status code between 400 and 599, the probe is considered failed.
