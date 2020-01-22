import React from 'react';
import {
    AppBar,
    Typography,
    Toolbar,
    Button,
    BottomNavigation,
    BottomNavigationAction,
} from '@material-ui/core';
import { makeStyles } from '@material-ui/core/styles';
import {
    LocationOn as LocationOnIcon,
    Restore as RestoreIcon,
    Favorite as FavoriteIcon,
} from '@material-ui/icons';

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
    const classes = useStyles();

    return (
        <div className={classes.root}>
            <AppBar position="static">
                <Toolbar>
                    {/* <IconButton */}
                    {/*     edge="start" */}
                    {/*     className={classes.menuButton} */}
                    {/*     color="inherit" */}
                    {/*     aria-label="menu"> */}
                    {/*     <MenuIcon /> */}
                    {/* </IconButton> */}
                    <Typography variant="h6" className={classes.title}>
                        Octo Budget
                    </Typography>
                    <Button color="inherit">Login</Button>
                </Toolbar>
            </AppBar>
            <BottomNavigation className={classes.stickToBottom}>
                <BottomNavigationAction label="Recents" value="recents" icon={<RestoreIcon />} />
                <BottomNavigationAction
                    label="Favorites"
                    value="favorites"
                    icon={<FavoriteIcon />}
                />
                <BottomNavigationAction label="Nearby" value="nearby" icon={<LocationOnIcon />} />
            </BottomNavigation>
        </div>
    );
};

export default App;
