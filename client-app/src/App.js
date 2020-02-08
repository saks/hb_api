import React from 'react';
import { Switch, Route, Link, Redirect, useLocation } from 'react-router-dom';
import { BottomNavigation, BottomNavigationAction } from '@material-ui/core';
import { makeStyles } from '@material-ui/core/styles';
import {
    BarChart as BudgetsIcon,
    FormatListBulleted as RecordsIcon,
    LocalOfferOutlined as TagsIcon,
} from '@material-ui/icons';

import Records from './components/Records';

import './App.css';

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
}));

const App = () => {
    const [currentPath, setCurrentPath] = React.useState('/');
    const currentLocation = useLocation();
    React.useEffect(
        () => {
            setCurrentPath(currentLocation.pathname);
        },
        [currentLocation]
    );
    const classes = useStyles();
    console.log('render...');

    return (
        <div className={classes.root}>
            <Route exact path="/tags" render={() => '...tags page'} />
            <Route exact path="/records" render={() => <Records />} />
            <Route exact path="/budgets" render={() => '...budgets page'} />
            <Switch>
                <Route path="/records/new" render={() => '...new record page'} />
                <Route
                    path="/records/:recordId"
                    render={({ match }) => {
                        const id = parseInt(match.params.recordId, 10);
                        return `...record ${id} page`;
                    }}
                />
            </Switch>
            <BottomNavigation
                className={classes.stickToBottom}
                value={currentPath}
                onChange={(_event, newPath) => {
                    setCurrentPath(newPath);
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
    );
};

export default App;
