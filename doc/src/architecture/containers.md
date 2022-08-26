# Containers

```plantuml
@startuml

{{#include ../../c4-plantuml/C4_Container.puml}}

Person(user, "User", "...")
Person(scout, "Scout", "...")

Boundary(browser, "Web Browser", "...") {
  Container_Ext(web_app, "Web Application", "...", "...")
  Container(ofdb_app_clearance, "Clearance Center", "...", "...")
  Container(ofdb_html, "OpenFairDB frontend", "...", "...")
}

Boundary(frontend_server, "Frontend Server", "...") {
  Container_Ext(kvm_server, "Server Application", "...", "...")
}

Boundary(server, "Server", "...") {
  Container(ofdb, "Open Fair DB", "...", "...")
  ContainerDb_Ext(db, "Local DB", "...", "...")
}

Rel(user, web_app, "uses")
Rel(scout, ofdb_app_clearance, "...")
Rel(scout, ofdb_html, "...")
Rel_U(kvm_server, web_app, "provides")
Rel(kvm_server, ofdb, "API requests")
Rel(ofdb_app_clearance, ofdb, "API requests")
Rel(ofdb, ofdb_html, "renders")
Rel(ofdb, db, "read/write")

@enduml
```
