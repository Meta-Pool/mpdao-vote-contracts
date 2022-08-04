import {
  Box,
  Button, 
  HStack, 
  Spacer, 
  Stack, 
  Text, 
  useBreakpointValue, 
  useDisclosure,
  VStack, 
} from '@chakra-ui/react';
import React, { useEffect } from 'react';
import { colors } from '../../../constants/colors';
import { getAvailableVotingPower, getBalanceMetaVote, getInUseVotingPower, getLockedBalance, getUnlockingBalance, withdrawAll } from '../../../lib/near';

import { useStore as useWallet } from "../../../stores/wallet";
import { useStore as useVoter } from "../../../stores/voter";
import { yton } from '../../../lib/util';
import LockModal from './LockModal';
import InfoModal from './InfoModal';
import { MODAL_TEXT } from '../../../constants';
import ButtonOnLogin from '../ButtonLogin';
import DashboardCard from './DashboardCard';

type Props = {
}

const DashboardHeader = () => {
  const { wallet} = useWallet();
  const { isOpen, onOpen, onClose } = useDisclosure();
  const { voterData, setVoterData } = useVoter();
  const { isOpen : infoIsOpen,  onClose : infoOnClose, onOpen: onOpenInfo} = useDisclosure();
  const isDesktop = useBreakpointValue({ base: false, md: true });

  const padding = '24px';

  const initMyData = async ()=> {
    const newVoterData = voterData;
    newVoterData.votingPower = await getAvailableVotingPower(wallet);
    newVoterData.inUseVPower = await getInUseVotingPower(wallet);
    newVoterData.metaLocked = await getLockedBalance(wallet);
    newVoterData.metaToWithdraw = await getBalanceMetaVote(wallet);
    newVoterData.metaUnlocking = await getUnlockingBalance(wallet);
    setVoterData(newVoterData);
  }

  const withdrawClicked = async ()=> {
       withdrawAll(wallet); 

  }

  useEffect(  () =>{
    (async ()=> {
      if (wallet && wallet.isSignedIn()) {
        initMyData()
      }
    })();
  },[wallet])

  return (
      <>
        <Stack 
          px={{base:'5px', md: '10%'}} 
          pb={{base:'32px', md: '150px'}} 
          borderBottomLeftRadius={{base:'32px', md: '0px'}} 
          borderBottomRightRadius={{base:'32px', md: '0px'}} 
          bg={colors.bgGradient} 
          w={'100%'} 
          flexDirection={{ base: 'column', md: 'column' }}  
          color={'white'} 
          spacing={'10px'} 
          justify={'space-between'}>
          <Stack justify={'space-between'} alignItems={'flex-start'} w={{ base: '100%'}}  spacing={10} p={padding} direction={'row'}>
            <HStack position={'relative'}>
              <VStack align={'flex-start'}>
                <Text hidden={!isDesktop} opacity={0.6} fontSize={'14px'} bg={"#120e2829"} p={'8px'}>My Voting Power</Text>
                <Text fontSize={{base: '32px', md: '64px'}} fontWeight={700} fontFamily={'Meta Space'} >{yton(voterData.votingPower)}</Text>
                <Text hidden={isDesktop} opacity={0.9} fontSize={'16px'}  p={'8px'}>My Voting Power</Text>
              </VStack>
              <Button borderRadius={100} disabled={!wallet?.isSignedIn()} position={'absolute'} px={5} top={0} right={0} onClick={onOpen}colorScheme={colors.primary}> +</Button>
            </HStack>
            <Stack top={3} position={'relative'} hidden={isDesktop}>
              <ButtonOnLogin>
                <Button borderRadius={100}  color={colors.primary} bg={'white'} fontSize={{ base: "xs", md: "xl" }}  onClick={onOpen} colorScheme={colors.secundary}>
                  Lock more $META
                </Button>
              </ButtonOnLogin>
            </Stack>
          </Stack>
          <Stack w={{ base: '100%', md: '100%' }} justifyContent={{base:'flex-end', md: 'space-between'}}  spacing={{base: 0, md: 5}} direction={'row'}>
            <HStack spacing={8}>
              <DashboardCard ligthMode={true} title='In use' iconSrc={'./icons/layer.png'} number={yton(voterData.inUseVPower)}></DashboardCard>
              <DashboardCard ligthMode={true} title='Projects  voted' iconSrc={'./icons/check.png'} number={voterData.votingResults.length}></DashboardCard>
            </HStack>
            <HStack spacing={8}>
              <Box hidden={!isDesktop}><DashboardCard   title='$META locked' iconSrc={'./icons/lock.png'} number={yton(voterData.metaLocked)}></DashboardCard> </Box>
              <Box hidden={!isDesktop}><DashboardCard   title='$META unlocking' iconSrc={'./icons/unlock.png'} number={yton(voterData.metaUnlocking)}></DashboardCard></Box>
              <Box hidden={!isDesktop} position={'relative'}>
                <DashboardCard  title='$META to withdraw' iconSrc={'./icons/withdraw.png'} number={yton(voterData.metaToWithdraw)}></DashboardCard>
                <Button minWidth= {'176px'} position={'absolute'} bottom={-14}  fontSize={'md'} p={6} borderRadius={100} disabled={ parseInt(voterData.metaToWithdraw)<=0}  onClick={()=> withdrawClicked()} color={colors.primary} bg={'white'} >
                  Withdraw
                </Button>
              </Box>
            </HStack>
          </Stack>
          <LockModal isOpen={isOpen} onClose={onClose} ></LockModal>
          <InfoModal content={MODAL_TEXT.UNLOCK} isOpen={infoIsOpen} onClose={infoOnClose} onSubmit={() => withdrawClicked()} ></InfoModal>
        </Stack>
        <Box  hidden={isDesktop}>
          <DashboardCard horizontal={true} title='$META locked' iconSrc={'./icons/lock_bold.png'} number={yton(voterData.metaLocked)}></DashboardCard>
          <DashboardCard horizontal={true} title='$META unlocking' iconSrc={'./icons/unlock_bold.png'} number={yton(voterData.metaUnlocking)}></DashboardCard>
          <DashboardCard horizontal={true} title='$META to withdraw' iconSrc={'./icons/withdraw_bold.png'} number={yton(voterData.metaToWithdraw)}></DashboardCard>
          <Button ml={'100px'} mt={5} maxWidth= {'88px'} h={'32px'} fontSize={'10px'} borderRadius={100} disabled={ parseInt(voterData.metaToWithdraw)<=0}  onClick={()=> withdrawClicked()} colorScheme={colors.primary} >
            Withdraw
          </Button>
        </Box>
      </>
  );
};

export default DashboardHeader;
 