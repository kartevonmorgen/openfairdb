# Context

The diagrom below shows the *general* context of the OpenFairDB system.

```plantuml
@startuml

{{#include ../../c4-plantuml/C4_Context.puml}}

HIDE_STEREOTYPE()

Person(user, "User")
Person(scout, "Scout")
System(ofdb, "OpenFairDB")

Boundary(applications, "Enduser Applications") {
  System_Ext(kvm_org, "Karte von morgen", "Web Application")
  System_Ext(mobile_app, "Mobile App", "iOS/Android")
}

Boundary(external_platforms, "External Platforms") {
  System_Ext(wordpress,"Wordpress System")
  System_Ext(wp_plugin,"Wordpress Plugin")
  System_Ext(custom_platform,"Custom Platform")

  Rel_D(wordpress, wp_plugin, "uses")
}

Boundary(external_services, "External Services") {
  System_Ext(mail_notification,"Mail Notification Service")
}

Rel(user, kvm_org, "uses")
Rel(user, mobile_app, "uses")
Rel(user, custom_platform, "uses")
Rel(user, wordpress, "uses")
Rel_L(scout, ofdb, "uses")
Rel_L(scout, kvm_org, "uses")

Rel(kvm_org, ofdb, "API requests", "HTTP")
Rel(mobile_app, ofdb, "API requests", "HTTP")

Rel_R(custom_platform, ofdb, "API requests", "HTTP")
Rel_R(wp_plugin, ofdb, "API requests", "HTTP")
Rel_U(mail_notification, ofdb, "API requests", "HTTP")

@enduml
```
