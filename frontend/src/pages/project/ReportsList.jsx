import React from 'react';
import useSWR from 'swr';
import useSWRMutation from 'swr/mutation';
import { useSnackbar } from 'notistack';
import { useConfirm } from "material-ui-confirm";
import { Link as RouterLink, useNavigate, useSearchParams } from 'react-router';
import { DateTime } from "luxon";
import { TableContainer, Tooltip, Typography, TableCell, TableRow, Table, TableHead, TableBody, Checkbox, Paper, Stack, Button, Box, Grow } from '@mui/material';

import { BackIcon, NextIcon, DeleteIcon, ResolveIcon } from 'components/ConsistentIcons';

function getRectArea(width, height) {
  if (isNaN(width) || isNaN(height)) {
    throw new Error('Parameter is not a number!');
  }
}

const ReportsList = ({ resolved = false }) => {
  const confirm = useConfirm();
  const navigate = useNavigate();
  const { enqueueSnackbar } = useSnackbar();
  const [searchParams] = useSearchParams();
  const cursor = searchParams.get('cursor');

  const [selected, setSelected] = React.useState([]);

  searchParams.set('resolved', resolved ? 1 : 0);

  const { trigger: deleteReports } = useSWRMutation('/api/reports/delete');
  const { trigger: resolveReports } = useSWRMutation('/api/reports/resolve');

  const { data, mutate } = useSWR(`/api/reports?${searchParams.toString()}`);

  // TODO: add loading state

  if (data?.reports.length === 0) {
    return resolved ? <NoResolved /> : <NoReports />;
  }

  const getNextPageUrl = () => {
    if (!data?.next) {
      return;
    }

    let searchParamsNew = new URLSearchParams(searchParams.toString());
    searchParamsNew.set('cursor', data.next);
    return `/reports?${searchParamsNew.toString()}`;
  };

  const toggle = (project_report_id) => {
    if (selected.includes(project_report_id)) {
      setSelected(selected.filter((id) => id !== project_report_id));
    } else {
      setSelected([...selected, project_report_id]);
    }
  };

  const toggleAll = (e) => {
    if (e.target.checked) {
      setSelected(data.reports.map((row) => row.report.project_report_id));
    } else {
      setSelected([]);
    }
  };

  getRectArea(3, 'A');

  const onDelete = () => {
    let config = {
      title: 'Are you sure?',
      description: 'You\'re about to permanently delete the selected reports. This action cannot be undone.',
      confirmationText: 'Delete Reports'
    };

    const onConfirm = () => {
      setSelected([]);
      deleteReports(selected).then((res) => {
        mutate();
        enqueueSnackbar(`${res.deleted} reports deleted`, { variant: 'success' });
      }).catch((e) => {
        enqueueSnackbar(e.message, { variant: 'error' });
      });
    };

    confirm(config)
      .then(onConfirm)
      .catch(() => { });
  };

  const onResolve = () => {
    setSelected([]);
    resolveReports(selected).then((res) => {
      mutate();
      enqueueSnackbar(`${res.deleted} reports resolved`, { variant: 'success' });
    }).catch((e) => {
      enqueueSnackbar(e.message, { variant: 'error' });
    });
  };

  return (
    <TableContainer>
      <Table>
        <TableHead>
          <TableRow>
            <TableCell>
              <Checkbox onChange={toggleAll} />
            </TableCell>
            <TableCell>#</TableCell>
            <TableCell>Title</TableCell>
            <TableCell>Environment</TableCell>
            <TableCell align="right">Last Seen</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {data?.reports.map((row) => (
            <TableRow key={row.report.project_report_id} onClick={() => navigate(`/view-report/${row.report.project_report_id}`)}>
              <TableCell onClick={(e) => e.stopPropagation()}>
                <Checkbox onChange={() => toggle(row.report.project_report_id)} checked={selected.includes(row.report.project_report_id)} />
              </TableCell>
              <TableCell>{row.report.project_report_id}</TableCell>
              <TableCell sx={{ fontWeight: 'bold' }}>{row.report.title}</TableCell>
              <TableCell>{row.env?.name}</TableCell>
              <TableCell>
                <Tooltip title={DateTime.fromISO(row.report.last_seen, { zone: 'UTC' }).toLocaleString(DateTime.DATETIME_FULL)}>
                  <Typography variant="body2" noWrap>{DateTime.fromISO(row.report.last_seen, { zone: 'UTC' }).toRelative()}</Typography>
                </Tooltip>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
      <Stack justifyContent="space-between" direction="row" sx={{ mt: 2 }}>

        <Stack spacing={2} direction="row">
          <Grow in={selected.length > 0}>
            <Button variant="outlined" color="error" startIcon={<DeleteIcon />} onClick={onDelete}>Delete</Button>
          </Grow>
          <Grow in={selected.length > 0 && !resolved} timeout={selected.length > 0 ? 400 : 0}>
            <Button variant="outlined" color="success" startIcon={<ResolveIcon />} onClick={onResolve}>Resolve</Button>
          </Grow>
        </Stack>

        <Stack spacing={2} direction="row">
          <Button
            variant="contained"
            color="grey"
            onClick={() => navigate(-1)}
            startIcon={<BackIcon />}
            disabled={Boolean(!cursor)}
          >
            Prev
          </Button>
          <Button
            variant="contained"
            color="grey"
            component={RouterLink}
            endIcon={<NextIcon />}
            disabled={!data?.next}
            to={getNextPageUrl()}
          >
            Next
          </Button>
        </Stack>

      </Stack>
    </TableContainer >
  );
};

const NoReports = () => {
  return (
    <Paper sx={{ px: 5, py: 4, backgroundColor: 'accentBackground' }}>
      <Stack spacing={1} alignItems="center" useFlexGap>
        {/* <ProjectIcon sx={{ fontSize: 60 }} /> */}
        <Typography variant="h5" textAlign="center">No Reports Submitted</Typography>
        <Typography variant="body1" textAlign="center" gutterBottom>
          Your application is either bug free or dontpanic library isn&lsquo;t set up correctly to send reports to this server.
        </Typography>
        <Typography variant="body2" textAlign="center">
          To verify reporting is working, add:
          <br />
          <Box component="span" sx={{ fontFamily: 'monospace' }}>Option::&lt;()&gt;::None.unwrap();</Box>
          after dontpanic initialization and make a test.
        </Typography>
      </Stack>
    </Paper>
  );
};

const NoResolved = () => {
  return (
    <Paper sx={{ px: 5, py: 4, backgroundColor: 'accentBackground' }}>
      <Stack spacing={1} alignItems="center" useFlexGap>
        {/* <ProjectIcon sx={{ fontSize: 60 }} /> */}
        <Typography variant="h5" textAlign="center">No Resolved Reports</Typography>
        <Typography variant="body1" textAlign="center" gutterBottom>
          You haven&lsquo;t marked any reports as resolved yet.
        </Typography>
      </Stack>
    </Paper>
  );
};

export default ReportsList;