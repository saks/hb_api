import React, { Suspense, useEffect } from 'react'
import createPersistedState from 'use-persisted-state'
import { Box, Grid } from '@material-ui/core'
import Pagination from '@material-ui/lab/Pagination'

const useCurrentPageState = createPersistedState('records_current_page')

const Records = props => {
    const [currentPage, setCurrentPage] = useCurrentPageState(0)
    const setTitle = props.setTitle

    useEffect(() => {
        setTitle('Records')
    }, [setTitle])

    return (
        <Box>
            <p>
                records page {currentPage}
                ...
            </p>
            <Suspense fallback={<h1>Loading posts...</h1>}>
                <ul>
                    {props.resource.read().map(post => (
                        <li key={post.id}>{post.text}</li>
                    ))}
                </ul>
            </Suspense>
            <Grid container justify="center">
                <Pagination count={5} size="large" />
            </Grid>
        </Box>
    )
}
export default Records
