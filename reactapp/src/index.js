import './import-jquery.js'
import './import-popper'
import './import-bootstrap'

import React from 'react'
import ReactDOM from 'react-dom'
import './index.css'
import App from './containers/App'
import * as serviceWorker from './serviceWorker'
import { Provider } from 'react-redux'
import store from './store'
import wasmModule from './wasm_loader'

import('./wasm/octo_budget_frontend').then(rust => {
    wasmModule.rust = rust

    ReactDOM.render(
        <Provider store={store}>
            <App />
        </Provider>,
        document.getElementById('root')
    )
})

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: http://bit.ly/CRA-PWA
serviceWorker.register({})
