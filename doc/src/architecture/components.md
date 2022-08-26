# Components

## Dependencies

The following diagram shows the dependencies of the components
and the association to the respective layers.

```plantuml
@startuml
{{#include ../../c4-plantuml/C4_Component.puml}}

System(ofdb, "OpenFairDB", "ofdb")

Boundary(infrastructure, "infrastructure") {
  Boundary(application, "application") {
    Boundary(domain, "domain") {
        Component(ofdb_core, "ofdb-core", "...")
        Component(ofdb_entities, "ofdb-entities", "...")
    }
    Component(ofdb_application, "ofdb-application", "...")
  }
  Component(ofdb_webserver, "ofdb-webserver", "...")
  Component(ofdb_db_sqlite, "ofdb-db-sqlite", "...")
  Component(ofdb_db_tantivy, "ofdb-db-tantivy", "...")
  Component(ofdb_gateways, "ofdb-db-gateways", "...")
  Component(ofdb_boundary, "ofdb-boundary", "...")
  Component(ofdb_app_clearance, "ofdb-app-clearance", "...")
  Component(ofdb_seed, "ofdb-seed", "...")
}

Rel(ofdb, ofdb_gateways, "...")
Rel(ofdb, ofdb_webserver, "...")

Rel(ofdb_core, ofdb_entities, "...")

Rel(ofdb_application, ofdb_core, "...")
Rel(ofdb_application, ofdb_entities, "...")
Rel(ofdb_application, ofdb_db_sqlite, "...")

Rel(ofdb_webserver, ofdb_application, "...")
Rel(ofdb_webserver, ofdb_app_clearance, "...")
Rel(ofdb_webserver, ofdb_boundary, "...")
Rel(ofdb_webserver, ofdb_core, "...")
Rel(ofdb_webserver, ofdb_entities, "...")
Rel(ofdb_webserver, ofdb_db_sqlite, "...")
Rel(ofdb_webserver, ofdb_db_tantivy, "...")

Rel(ofdb_app_clearance, ofdb_seed, "...")

@enduml
```

## Domain Model

```plantuml
@startuml
{{#include classes.puml}}
@enduml
```
