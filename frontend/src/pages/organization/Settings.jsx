import React from 'react';
import useSWRMutation from 'swr/mutation';
import * as yup from "yup";
import { useNavigate } from 'react-router';
import { useSnackbar } from 'notistack';
import { useForm, FormProvider } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import { Box, Divider, Grid2 as Grid, Stack, Typography, Tab } from '@mui/material';
import { LoadingButton, TabList, TabContext, TabPanel } from "@mui/lab";

import SideMenu from 'components/SideMenu';
import { FormServerError, ControlledTextField } from "components/form";
import { SaveIcon } from 'components/ConsistentIcons';

const Settings = () => {
  return (
    <>Settings</>
  );
};

export default Settings;