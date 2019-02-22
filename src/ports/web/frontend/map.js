(function(){
  var TILES = 'https://maps.wikimedia.org/osm-intl/{z}/{x}/{y}{r}.png';
  var all_available = function(names){
    for(var i=0;i<names.length;i++) {
      if (!window[names[i]]) {
        return false;
      }
    }
    return true;
  };

  var ready = function(names, callback) {
    var interval = 10; // ms
    window.setTimeout(function() {
        if (all_available(names)) {
          var vars = [];
          for(var i=0;i<names.length;i++) {
            vars.push(window[names[i]]);
          }
          callback.apply(null, vars);
        } else {
            window.setTimeout(arguments.callee, interval);
        }
    }, interval);
  };

  ready(["OFDB_MAP_PINS", "OFDB_MAP_ZOOM", "OFDB_MAP_CENTER", "L"],
    function(pins, zoom, center, L){
    if (pins.length > 0){
      var map = L.map('map').setView(center,zoom);
      L.tileLayer(TILES, { attribution: 'slowtec GmbH', maxZoom: 18 }).addTo(map);
      for(var i=0;i<pins.length;i++) {
        var pin = pins[i];
        L.marker([pin.lat,pin.lng]).addTo(map);
      }
    }
  });
})();
