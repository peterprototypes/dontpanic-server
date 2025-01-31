import React from 'react';
import useSWR from 'swr';
import useSWRMutation from 'swr/mutation';
import { useSnackbar } from 'notistack';
import { useConfirm } from "material-ui-confirm";
import { Link as RouterLink, useNavigate, useSearchParams } from 'react-router';
import { DateTime } from "luxon";
import { TableContainer, Tooltip, Typography, TableCell, TableRow, Table, TableHead, TableBody, Checkbox, Paper, Stack, Box, Grow, LinearProgress, IconButton, Link } from '@mui/material';
import { styled } from '@mui/system';
import { LoadingButton } from '@mui/lab';

import { BackIcon, NextIcon, DeleteIcon, ResolveIcon } from 'components/ConsistentIcons';
import LoadingPage from 'components/LoadingPage';

const ReportsList = ({ resolved = false }) => {
  const confirm = useConfirm();
  const navigate = useNavigate();
  const { enqueueSnackbar } = useSnackbar();
  const [searchParams] = useSearchParams();
  const cursor = searchParams.get('cursor');

  const [selected, setSelected] = React.useState([]);

  searchParams.set('resolved', resolved ? 1 : 0);

  const { trigger: deleteReports, isMutating: isDeleting } = useSWRMutation('/api/reports/delete');
  const { trigger: resolveReports, isMutating: isResolving } = useSWRMutation('/api/reports/resolve');
  const { data, mutate, isValidating, isLoading } = useSWR(`/api/reports?${searchParams.toString()}`);

  React.useEffect(() => {
    // Only subscribe if we're on the first page
    if (cursor) return;

    const api_url = import.meta.env.DEV ? "http://localhost:8080" : "";

    const eventSource = new EventSource(api_url + `/api/reports/subscribe?${searchParams.toString()}`, { withCredentials: true });

    eventSource.onmessage = () => {
      // Trigger SWR to revalidate the data
      mutate();
    };

    eventSource.onerror = (error) => {
      console.error('EventSource failed:', error);
      eventSource.close();
    };

    return () => {
      eventSource.close();
    };
  }, [cursor, searchParams, mutate]);

  if (isLoading) {
    return <LoadingPage />;
  }

  if (data.reports.length === 0) {
    return resolved ? <NoResolved /> : <NoReports project={data?.project} />;
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

  const onDelete = () => {
    let config = {
      title: 'Confirm Deletion',
      description: 'This action will permanently delete the selected reports and cannot be undone. Do you wish to proceed?',
      confirmationText: 'Delete Reports'
    };

    const onConfirm = () => {
      deleteReports(selected).then((res) => {
        mutate();
        enqueueSnackbar(`${res.deleted} reports deleted`, { variant: 'success' });
        setSelected([]);
      }).catch((e) => {
        enqueueSnackbar(e.message, { variant: 'error' });
      });
    };

    confirm(config)
      .then(onConfirm)
      .catch(() => { });
  };

  const onResolve = () => {
    resolveReports(selected).then((res) => {
      mutate();
      enqueueSnackbar(`${res.deleted} reports resolved`, { variant: 'success' });
      setSelected([]);
    }).catch((e) => {
      enqueueSnackbar(e.message, { variant: 'error' });
    });
  };

  return (
    <TableContainer>
      {isValidating ? <LinearProgress /> : <Box sx={{ height: 4 }} />}
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
          {data.reports.map((row) => (
            <ReportRow key={row.report.project_report_id} onClick={() => navigate(`/view-report/${row.report.project_report_id}`)}>
              <TableCell onClick={(e) => e.stopPropagation()}>
                <Checkbox onChange={() => toggle(row.report.project_report_id)} checked={selected.includes(row.report.project_report_id)} />
              </TableCell>
              <TableCell>{row.report.project_report_id}</TableCell>
              <TableCell sx={{ fontWeight: row.report.is_seen ? 'normal' : 'bold' }}>{row.report.title}</TableCell>
              <TableCell>{row.env?.name}</TableCell>
              <TableCell>
                <Tooltip title={DateTime.fromISO(row.report.last_seen, { zone: 'UTC' }).toLocaleString(DateTime.DATETIME_FULL)}>
                  <Typography variant="body2" noWrap>{DateTime.fromISO(row.report.last_seen, { zone: 'UTC' }).toRelative()}</Typography>
                </Tooltip>
              </TableCell>
            </ReportRow>
          ))}
        </TableBody>
      </Table>
      <Stack justifyContent="space-between" direction="row" sx={{ mt: 2 }}>

        <Stack spacing={2} direction="row">
          <Grow in={selected.length > 0}>
            <LoadingButton
              variant="outlined"
              color="error"
              startIcon={<DeleteIcon />}
              onClick={onDelete}
              loading={isDeleting}
            >
              Delete Selected
            </LoadingButton>
          </Grow>
          <Grow in={selected.length > 0 && !resolved} timeout={selected.length > 0 ? 400 : 0}>
            <LoadingButton
              variant="outlined"
              color="success"
              startIcon={<ResolveIcon />}
              onClick={onResolve}
              loading={isResolving}
            >
              Mark as Resolved
            </LoadingButton>
          </Grow>
        </Stack>

        <Stack spacing={2} direction="row">
          <Tooltip title="Previous Page">
            <span>
              <IconButton
                variant="contained"
                color="grey"
                onClick={() => navigate(-1)}
                disabled={Boolean(!cursor)}
              >
                <BackIcon />
              </IconButton>
            </span>
          </Tooltip>
          <Tooltip title="Next Page">
            <span>
              <IconButton
                variant="contained"
                color="grey"
                component={RouterLink}
                disabled={!data?.next}
                to={getNextPageUrl()}
              >
                <NextIcon />
              </IconButton>
            </span>
          </Tooltip>
        </Stack>

      </Stack>
    </TableContainer >
  );
};

const NoReports = ({ project }) => {
  return (
    <Stack spacing={2}>
      <Paper sx={{ px: 5, py: 4, backgroundColor: 'accentBackground' }}>
        <Stack spacing={2} useFlexGap>
          <Typography variant="h5" textAlign="center">No Reports Found</Typography>
          <Typography variant="body2" textAlign="center" color="textSecondary" gutterBottom>
            It looks like your application is running smoothly. If you expected reports here, ensure reporting is properly configured.
          </Typography>
        </Stack>
      </Paper>

      {project && <IntegrationExample project={project} />}
    </Stack>
  );
};

const IntegrationExample = ({ project }) => {
  return (
    <Box>
      <Typography variant="h5" gutterBottom>How to Integrate with {project.name}</Typography>
      <Typography variant="body2" color="textSecondary" gutterBottom>
        Follow the steps below to integrate <strong>Don&lsquo;t Panic</strong> with your project.
      </Typography>

      <Typography variant="subtitle1" sx={{ mt: 3 }}>Step 1: Install the Dependency</Typography>

      <Paper sx={{ px: 3, py: 2, backgroundColor: '', fontFamily: 'monospace', mb: 2 }}>
        <Box component="span" sx={{ color: 'success.main' }}>user@host</Box>
        {':'}
        <Box component="span" sx={{ color: 'primary.main' }}>~/myapp</Box>
        {'$ cargo add dontpanic'}
      </Paper>

      <Typography variant="subtitle1" sx={{ mt: 3 }}>Step 2: Initialize the Library</Typography>
      <Paper component="pre" sx={{ px: 3, py: 2, m: 0, backgroundColor: '', fontFamily: 'monospace' }}>
        <Box component="span" sx={{ color: 'info.dark' }}>fn</Box>
        {' '}
        <Box component="span" sx={{ color: 'secondary.main' }}>main</Box>
        {'() -> '}
        <Box component="span" sx={{ color: 'primary.main' }}>Result</Box>
        {'<(), '}
        <Box component="span" sx={{ color: 'primary.main' }}>Box</Box>
        {'<'}
        <Box component="span" sx={{ color: 'info.dark' }}>dyn</Box>
        {' '}
        <Box component="span" sx={{ color: 'primary.main' }}>Error</Box>
        {'>> {'}
        <br />
        {'    '}
        <Box component="span" sx={{ color: 'primary.main' }}>dontpanic</Box>
        {'::'}
        <Box component="span" sx={{ color: 'secondary.main' }}>builder</Box>
        {'('}
        <Box component="span" sx={{ color: 'error.main' }}>&quot;{project.api_key}&quot;</Box>
        {')'}
        <br />
        {'        .'}
        <Box component="span" sx={{ color: 'secondary.main' }}>version</Box>
        {'('}
        <Box component="span" sx={{ color: 'info.dark' }}>env!</Box>
        {'('}
        <Box component="span" sx={{ color: 'error.main' }}>&quot;CARGO_PKG_VERSION&quot;</Box>
        {')'}
        {')'}
        <br />
        {'        .'}
        <Box component="span" sx={{ color: 'secondary.main' }}>build</Box>
        {'()?;'}
        <br /><br />
        {'    '}
        <Box component="span" sx={{ color: 'success.main' }}>{'// Your application logic here'}</Box>
        <br />
        {'}'}
      </Paper>

      {/* Step 3: Trigger a Test Report */}
      <Typography variant="subtitle1" sx={{ mt: 3 }}>Step 3: Verify Integration</Typography>
      <Typography variant="body2" color="textSecondary">
        To confirm that error reporting is working, you can trigger a test report by adding the following code snippet:
      </Typography>
      <Paper component="pre" sx={{ px: 3, py: 2, backgroundColor: 'background.default', fontFamily: 'monospace' }}>
        <Box component="span" sx={{ color: 'primary.main' }}>Option</Box>
        {'::<()>::'}
        <Box component="span" sx={{ color: 'primary.main' }}>None</Box>
        {'.'}
        <Box component="span" sx={{ color: 'secondary.main' }}>expect</Box>
        {'('}
        <Box component="span" sx={{ color: 'error.main' }}>&quot;Reporting is working&quot;</Box>
        {');'}
      </Paper>

      {/* Documentation Link */}
      <Typography variant="body2" sx={{ mt: 3 }}>
        For more details, visit the <Link href="https://docs.rs/dontpanic/latest/dontpanic" target="_blank">documentation</Link>.
      </Typography>
    </Box>
  );
};

const NoResolved = () => {
  return (
    <Paper sx={{ px: 5, py: 4, backgroundColor: 'accentBackground' }}>
      <Stack spacing={1} alignItems="center" useFlexGap>
        <Typography variant="h5" textAlign="center">No Resolved Reports</Typography>
        <Typography variant="body1" textAlign="center" gutterBottom>
          No reports have been marked as resolved yet. You can resolve issues from the reports list.
        </Typography>
      </Stack>
    </Paper>
  );
};

const ReportRow = styled(TableRow)(({ theme }) => ({
  cursor: 'pointer',
  '&:hover': {
    backgroundColor: theme.palette.action.hover
  }
}));

export default ReportsList;