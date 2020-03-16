// @flow

export const getAuthToken = async (username: string, password: string) => {
    const formData = { username, password }
    const response = await fetch('/auth/jwt/create/', {
        method: 'POST',
        body: JSON.stringify(formData),
        headers: {
            'User-Agent': 'Home Budget PWA',
            'Content-Type': 'application/json',
        },
    })

    if (response.ok) {
        const data = await response.json()
        return data.token
    } else if (400 === response.status) {
        // validation errors
        const data = await response.json()
        // TODO: display validation errors
    } else if (404 === response.status) {
        // username or password is not valid
        // TODO: display failed auth error
    }
}

export const fetchUserData = (token: string) => {
    let tagsPromise = fetchTags(token)
    let recordsPromise = fetchRecords(token)
    let budgetsPromise = fetchBudgets(token)
    return {
        tags: wrapPromise(tagsPromise),
        records: wrapPromise(recordsPromise),
        budgets: wrapPromise(budgetsPromise),
    }
}

const fetchWithAuth = async (token: string) => {
    // TODO
}

const fetchTags = async (token: string) => {}
const fetchRecords = (token: string) => {
    return new Promise(resolve => {
        setTimeout(() => {
            console.log('fetched posts')
            resolve([
                {
                    id: 0,
                    text: 'I get by with a little help from my friends',
                },
                {
                    id: 1,
                    text: "I'd like to be under the sea in an octupus's garden",
                },
                {
                    id: 2,
                    text: 'You got that sand all over your feet',
                },
            ])
        }, 2000)
    })
}
const fetchBudgets = async (token: string) => {}

function wrapPromise(promise) {
    let status = 'pending'
    let result
    let suspender = promise.then(
        r => {
            status = 'success'
            result = r
        },
        e => {
            status = 'error'
            result = e
        }
    )
    return {
        read() {
            if (status === 'pending') {
                throw suspender
            } else if (status === 'error') {
                throw result
            } else if (status === 'success') {
                return result
            }
        },
    }
}
