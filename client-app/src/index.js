// @flow

import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import App from './App';
import * as serviceWorker from './serviceWorker';
import wasmModule from './wasm_loader';

const root = document.getElementById('root');
if (root) {
    import('./wasm').then(rust => {
        wasmModule.rust = rust;

        ReactDOM.render(<App />, root);
    });
}

serviceWorker.register();
