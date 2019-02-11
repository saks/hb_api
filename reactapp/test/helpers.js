import { mount, render, shallow, configure } from 'enzyme'
import Adapter from 'enzyme-adapter-react-16'

// load wasm module
import wasm_module from './../src/wasm_loader'
import('./../src/test_wasm/octo_budget_frontend').then(
    rust => {
        wasm_module.rust = rust
    },
    e => {
        console.log('ERRR!!!!')
        console.log(e)
    }
)
// finish load wasm module

configure({ adapter: new Adapter() })

global.expect = expect

global.mount = mount
global.render = render
global.shallow = shallow
