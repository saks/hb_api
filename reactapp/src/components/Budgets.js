// @flow

import React, { Component } from 'react'

import Budget from './Budget'
import type { Attrs } from '../types/Budget'

export default class Budgets extends Component<{| list: Array<Attrs> |}> {
    get budgets() {
        const list = this.props.list.map(attrs => <Budget attrs={attrs} key={attrs.name} />)

        return <div id="budget-cards">{list}</div>
    }

    render() {
        return (
            <div id="budgets">
                <div className="row justify-content-center">
                    <h2>Budgets</h2>
                </div>
                {this.budgets}
            </div>
        )
    }
}
