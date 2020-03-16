import React, { useEffect } from 'react'

const Budgets = props => {
    const setTitle = props.setTitle

    useEffect(() => {
        setTitle('Budgets')
    }, [setTitle])

    return <p>budgets page...</p>
}
export default Budgets
