import 'leaflet/dist/leaflet.css';
  import L from 'leaflet';
  import './style.css';
  import init, { parse_and_process_csv } from './wasm/wasm_pkg';

  async function main() {
    await init();

    // Setup the application UI
    document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
      <h1>CSV Map Drop Demo (Rust/WASM parse)</h1>
      <div id="drop-area" style="border:2px dashed #888;padding:2em;margin:2em 0;text-align:center">
        <b>Drop your CSV here</b>
        <div><small>Format: lon,lat,text per line</small>
        <br>
        <input type="file" id="csv-file" accept=".csv" style="margin-top:1em" /></div>
      </div>
      <div id="map" style="height: 480px;"></div>
    `;

    // Initialize map
    const map = L.map('map').setView([48.8584, 2.2945], 3);
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '© OpenStreetMap contributors',
    }).addTo(map);

    // Removes all markers from the map (but keeps the map/tileLayer)
    function clearMarkers() {
      map.eachLayer(layer => {
        // TileLayer and other system layers have a getAttribution function
        if (!(layer instanceof L.TileLayer)) {
          map.removeLayer(layer);
        }
      });
    }

    // Render the markers given the Rust/WASM-processed JSON array
    function renderMarkers(json: string) {
      let results: { lon: number, lat: number, processed: string }[] = [];
      try {
        results = JSON.parse(json);
      } catch {
        alert('Failed to parse WASM output');
        return;
      }

      if (results.length === 0) {
        alert('No valid coordinates found in CSV.');
        return;
      }

      clearMarkers();

      for (const { lon, lat, processed } of results) {
        const marker = L.marker([lat, lon]).addTo(map);
        marker.bindPopup(`<b>${processed}</b>`);
        marker.on('click', () => marker.openPopup());
      }

      // Center map on first coordinate
      map.setView([results[0].lat, results[0].lon], 5);
    }

    // Handle CSV as string and call into Rust
    function processCSVText(csvText: string) {
      // Call the Rust function
      const jsonResult = parse_and_process_csv(csvText);
      renderMarkers(jsonResult);
    }

    // Drag & drop area events
    const dropArea = document.getElementById('drop-area')!;
    ['dragenter', 'dragover'].forEach(event =>
      dropArea.addEventListener(event, e => {
        e.preventDefault();
        dropArea.classList.add('dragging');
      })
    );
    ['dragleave', 'drop'].forEach(event =>
      dropArea.addEventListener(event, e => {
        e.preventDefault();
        dropArea.classList.remove('dragging');
      })
    );
    dropArea.addEventListener('drop', e => {
      e.preventDefault();
      dropArea.classList.remove('dragging');
      const files = (e as DragEvent).dataTransfer?.files;
      if (files && files[0]) {
        readFile(files[0]);
      }
    });

    // File input for manual selection:
    document.getElementById('csv-file')!.addEventListener('change', (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) readFile(file);
    });

    function readFile(file: File) {
      const reader = new FileReader();
      reader.onload = function () {
        if (typeof reader.result === 'string') {
          processCSVText(reader.result);
        }
      };
      reader.readAsText(file);
    }
  }

  main();