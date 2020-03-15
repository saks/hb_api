import React from 'react'
import {
    makeStyles,
    TextField,
    InputAdornment,
    Button,
    IconButton,
    Table,
    Typography,
} from '@material-ui/core'
import TableBody from '@material-ui/core/TableBody'
import TableCell from '@material-ui/core/TableCell'
import TableContainer from '@material-ui/core/TableContainer'
import TableRow from '@material-ui/core/TableRow'
import Paper from '@material-ui/core/Paper'
import AddIcon from '@material-ui/icons/Add'

const useStyles = makeStyles(theme => ({
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
const rows = ['Foo', 'Bar']

const Tags = props => {
    props.setTitle('Tags')
    const classes = useStyles()

    const handleClickShowPassword = () => {}
    const handleMouseDownPassword = () => {}

    return (
        <div>
            <form noValidate autoComplete="off">
                <TextField
                    endAdornment={
                        <InputAdornment position="end">
                            <IconButton aria-label="toggle password visibility" edge="end">
                                <AddIcon />
                            </IconButton>
                        </InputAdornment>
                    }
                />
            </form>
            <TableContainer component={Paper}>
                <Table aria-label="simple table">
                    <TableBody>
                        {rows.map(tag => (
                            <TableRow key={tag}>
                                <TableCell component="th" scope="tag">
                                    {tag}
                                </TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
            </TableContainer>
        </div>
    )
}
export default Tags
