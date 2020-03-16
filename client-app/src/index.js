// @flow

import React, { Suspense } from 'react'
import ReactDOM from 'react-dom'
import './index.css'
import App from './App'
import { HashRouter as Router } from 'react-router-dom'
import * as serviceWorker from './serviceWorker'
import wasmModule from './wasm_loader'

const rootElement = document.getElementById('root')
import('./wasm').then(rust => {
    wasmModule.rust = rust

    ReactDOM.createRoot(rootElement).render(
        <Router>
            <App />
        </Router>
    )
})

serviceWorker.register()
