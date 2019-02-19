(function(){
  if (window.OFDB_EVENT_POS){
    var lat = OFDB_EVENT_POS.lat;
    var lng = OFDB_EVENT_POS.lng;
    var map = L.map('map').setView([lat, lng],13);
    L.tileLayer('https://maps.wikimedia.org/osm-intl/{z}/{x}/{y}{r}.png', {
      attribution: 'slowtec GmbH',
      maxZoom: 18 
    }).addTo(map);
    var marker = L.marker([lat,lng]).addTo(map);
  }
})();
