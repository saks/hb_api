// @flow

import type { State, Action } from '../types/Budget'

const defaultState: State = {
    list: [],
    isFetching: false,
}

// TODO: add tests
const sortBudgets = list => {
    const indexOfTotal = list.findIndex(budget => budget.name === 'Total')

    if (indexOfTotal < 0) {
        return list
    }

    const totalBudget = list.splice(indexOfTotal, 1)[0]

    if (totalBudget === undefined) {
        return list
    }

    list.unshift(totalBudget)

    return list
}

export default (state: State = defaultState, action: Action) => {
    switch (action.type) {
        case 'START_LOADING_BUDGETS_PAGE':
            return { ...state, isFetching: true }
        case 'FINIS_LOADING_BUDGETS_PAGE':
            return { ...state, isFetching: false }
        case 'SET_LIST_FOR_BUDGETS_PAGE':
            const list = sortBudgets(action.list)
            return { ...state, list }
        default:
            return state
    }
}
