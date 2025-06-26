import './style.css';
import tileart from './tileart/zac/index.js';
import { init, render_map } from './pkg';

init();
render_map(tileart);