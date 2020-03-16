import React, { useRef, useState } from 'react'
import { blue } from '@material-ui/core/colors'
import {
    Dialog,
    DialogTitle,
    DialogContent,
    DialogActions,
    Button,
    TextField,
    Slide,
    CircularProgress,
    makeStyles,
} from '@material-ui/core'
import { getAuthToken } from '../lib/http_client'

const Transition = React.forwardRef(function Transition(props, ref) {
    return <Slide direction="down" ref={ref} {...props} />
})

const useStyles = makeStyles(theme => ({
    wrapper: {
        margin: theme.spacing(1),
        position: 'relative',
    },
    buttonProgress: {
        color: blue[500],
        position: 'absolute',
        top: '50%',
        left: '50%',
        marginTop: -12,
        marginLeft: -12,
    },
}))

export default props => {
    const classes = useStyles()
    const needSignIn = props.authToken === null
    const [isOpen, setIsOpen] = useState(needSignIn)
    const [isLoading, setIsLoading] = useState(false)

    const usernameRef = useRef()
    const passRef = useRef()

    const handleStartLogin = () => setIsOpen(true)
    const handleClose = () => setIsOpen(false)
    const handleSubmit = async () => {
        setIsLoading(true)

        const username = usernameRef.current.value
        const pass = passRef.current.value

        try {
            const authToken = await getAuthToken(username, pass)
            if (authToken) {
                props.setAuthToken(authToken)
                setIsLoading(false)
                setIsOpen(false)
            }
        } catch (err) {
            console.log(err)
        }
    }
    return (
        <div>
            {needSignIn && (
                <Button color="inherit" onClick={handleStartLogin}>
                    Login
                </Button>
            )}
            <Dialog
                keepMounted
                open={isOpen}
                TransitionComponent={Transition}
                onClose={handleClose}
                aria-labelledby="form-dialog-title"
            >
                <DialogTitle id="form-dialog-title">
                    {isLoading ? 'Sign in...' : 'Sign in'}
                </DialogTitle>
                <DialogContent>
                    <TextField
                        required
                        fullWidth
                        disabled={isLoading}
                        inputRef={usernameRef}
                        type="username"
                        id="username"
                        label="Username"
                        defaultValue=""
                        placeholder=""
                        margin="normal"
                        InputLabelProps={{ shrink: true }}
                    />
                    <TextField
                        required
                        fullWidth
                        disabled={isLoading}
                        inputRef={passRef}
                        autoComplete="off"
                        type="password"
                        id="password"
                        label="Password"
                        defaultValue=""
                        placeholder=""
                        margin="normal"
                        InputLabelProps={{ shrink: true }}
                    />
                </DialogContent>
                <DialogActions>
                    <Button disabled={isLoading} onClick={handleClose}>
                        Cancel
                    </Button>
                    <div className={classes.wrapper}>
                        <Button
                            disabled={isLoading}
                            onClick={handleSubmit}
                            type="submit"
                            color="primary"
                        >
                            Sign In
                        </Button>
                        {isLoading && (
                            <CircularProgress size={24} className={classes.buttonProgress} />
                        )}
                    </div>
                </DialogActions>
            </Dialog>
        </div>
    )
}
