import useSWRMutation from 'swr/mutation';
import * as yup from "yup";
import { useNavigate } from 'react-router';
import { useSnackbar } from 'notistack';
import { useForm, FormProvider } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import { Box, Divider, Grid2 as Grid, Stack, Typography } from '@mui/material';
import { LoadingButton } from "@mui/lab";

import SideMenu from 'components/SideMenu';
import { FormServerError, ControlledTextField } from "components/form";
import { SaveIcon } from 'components/ConsistentIcons';

const Projects = () => {
  return (
    <Grid container spacing={4}>
      <Grid size={3}>
        <SideMenu />
      </Grid>
      <Grid size={9}>
        Projects
      </Grid>
    </Grid>
  );
};

export default Projects;