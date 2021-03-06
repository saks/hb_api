// @flow

import React, { Component } from 'react'
import * as moment from 'moment-timezone'

import { EXP } from '../constants/TransactionTypes'
import RecordModel from '../models/Record'
import { fmtNum } from '../utils'

import type { RouterHistory } from 'react-router-dom'

const DATETIME_FORMAT_OPTIONS = {
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
}

type Props = {|
    +model: RecordModel,
    +history: RouterHistory,
|}

export default class Record extends Component<Props, void> {
    get amount(): string {
        return fmtNum(this.props.model.amount)
    }

    get className(): string {
        const suffix = this.props.model.transaction_type === EXP ? 'warning' : 'success'
        return `card record-item bd-callout bd-callout-${suffix}`
    }

    get tags(): string {
        return Array.from(this.props.model.tags).join(', ')
    }

    get date(): string {
        const tz = moment.tz.guess()
        const momentObject = moment.tz(this.props.model.created_at * 1000, tz)

        return momentObject.toDate().toLocaleString('en', DATETIME_FORMAT_OPTIONS)
    }

    get comment(): string {
        return this.props.model.comment
    }

    edit(): void {
        if (this.props.model.id) {
            this.props.history.push(`/records/${this.props.model.id}`)
        }
    }

    render() {
        return (
            <div className={this.className} onClick={this.edit.bind(this)}>
                <div className="card-body">
                    <h5 className="text-center">
                        <span className="amount font-weight-bold float-left">{this.amount}</span>
                        <small className="date">{this.date}</small>
                        <span className="tags badge badge-info float-right">{this.tags}</span>
                    </h5>
                    <small className="comment float-left text-center font-weight-light">
                        {this.comment}
                    </small>
                </div>
            </div>
        )
    }
}
