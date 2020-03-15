import React from 'react'
import createPersistedState from 'use-persisted-state'
import { Box, Grid } from '@material-ui/core'
import Pagination from '@material-ui/lab/Pagination'

const useCurrentPageState = createPersistedState('records_current_page')

const Records = props => {
    props.setTitle('Records')

    const [currentPage, setCurrentPage] = useCurrentPageState(0)
    return (
        <Box>
            <p>
                records page {currentPage}
                ...
            </p>
            <Grid container justify="center">
                <Pagination count={5} size="large" />
            </Grid>
        </Box>
    )
}
export default Records
