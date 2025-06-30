import './style.css';
import tileart from './tileart/zac/index.js';
import { init, render_map } from './pkg';

init();
render_map(tileart);

var refreshButton = document.getElementById("refresh");
refreshButton.addEventListener("click", function() {
    render_map(tileart);
});

var downloadButton = document.getElementById("download");
downloadButton.addEventListener("click", function() {
    let canvas = document.getElementById('canvas');
    // Create a Blob from the canvas data
    canvas.toBlob((blob) => {
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'map.png';
        a.click();
        URL.revokeObjectURL(url);
    }, 'image/png');
});