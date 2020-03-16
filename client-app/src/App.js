import React, { Suspense, useState } from 'react'
import { Switch, Route, Link, Redirect, useLocation } from 'react-router-dom'
import MenuIcon from '@material-ui/icons/Menu'
import { AppBar, IconButton, Toolbar, Typography } from '@material-ui/core'
import { Container, BottomNavigation, BottomNavigationAction } from '@material-ui/core'
import { makeStyles } from '@material-ui/core/styles'
import {
    BarChart as BudgetsIcon,
    FormatListBulleted as RecordsIcon,
    LocalOfferOutlined as TagsIcon,
} from '@material-ui/icons'

import Records from './components/Records'
import Budgets from './components/Budgets'
import Tags from './components/Tags'
import AuthDialog from './components/AuthDialog'
import { fetchUserData } from './lib/http_client'
import './App.css'

const resource = fetchUserData()

const useStyles = makeStyles(theme => ({
    stickToBottom: {
        width: '100%',
        position: 'fixed',
        bottom: 0,
    },
    root: {
        flexGrow: 1,
    },
    menuButton: {
        marginRight: theme.spacing(2),
    },
    title: {
        flexGrow: 1,
    },
}))

const App = () => {
    const [authToken, setAuthToken] = useState(null)
    const [currentTitle, setTitle] = useState('Octo Budget')
    const [currentPath, setCurrentPath] = useState('/')
    const currentLocation = useLocation()
    React.useEffect(() => {
        setCurrentPath(currentLocation.pathname)
    }, [currentLocation])
    const classes = useStyles()

    return (
        <div className={classes.root}>
            <AppBar position="static">
                <Toolbar>
                    <IconButton
                        edge="start"
                        className={classes.menuButton}
                        color="inherit"
                        aria-label="menu"
                    >
                        <MenuIcon />
                    </IconButton>
                    <Typography variant="h6" className={classes.title}>
                        {currentTitle}
                    </Typography>
                    <AuthDialog setAuthToken={setAuthToken} authToken={authToken} />
                </Toolbar>
            </AppBar>
            <Container maxWidth="sm">
                <Route exact path="/tags">
                    <Tags setTitle={setTitle} resource={resource.tags} />
                </Route>
                <Route exact path="/records">
                    <Suspense fallback={<h1>Loading records...</h1>}>
                        <Records setTitle={setTitle} resource={resource.records} />
                    </Suspense>
                </Route>
                <Route exact path="/budgets">
                    <Budgets setTitle={setTitle} resource={resource.budgets} />
                </Route>
                <Switch>
                    <Route path="/records/new" render={() => '...new record page'} />
                    <Route
                        path="/records/:recordId"
                        render={({ match }) => {
                            const id = parseInt(match.params.recordId, 10)
                            return `...record ${id} page`
                        }}
                    />
                </Switch>
            </Container>
            <BottomNavigation
                className={classes.stickToBottom}
                value={currentPath}
                onChange={(_event, newPath) => {
                    setCurrentPath(newPath)
                }}
            >
                <BottomNavigationAction
                    component={Link}
                    to="/records"
                    label="Records"
                    value="/records"
                    icon={<RecordsIcon />}
                />
                <BottomNavigationAction
                    component={Link}
                    to="/budgets"
                    label="Budgets"
                    value="/budgets"
                    icon={<BudgetsIcon />}
                />
                <BottomNavigationAction
                    component={Link}
                    to="/tags"
                    label="Tags"
                    value="/tags"
                    icon={<TagsIcon />}
                />
            </BottomNavigation>
            <Route exact path="/" render={() => <Redirect to="/records" />} />
        </div>
    )
}

export default App
